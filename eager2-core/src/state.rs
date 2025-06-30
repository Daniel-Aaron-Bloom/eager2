use std::mem;

use crate::{
    consts::{EAGER_SIGIL, LAZY_SIGIL},
    egroup::{EfficientGroupT, EfficientGroupV},
    exec::ExecutableMacroType,
    parse::{
        expect_call_literal, expect_group, expect_mode, MacroPathSegments, MacroPathType, Param,
    },
    pm::{token_stream, Delimiter, Group, Ident, Literal, Span, ToTokens, TokenStream, TokenTree},
    utils::{eager_data, NextOr, PopNext},
    Error,
};

/// Mode is the recordable state of the macro execution environment.
///
/// It can either be eager or lazy, represented by `EAGER_SIGIL` and `LAZY_SIGIL` respectively.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Eager,
    Lazy,
}

impl TryFrom<&'_ str> for Mode {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(match s {
            "eager" => Mode::Eager,
            "lazy" => Mode::Lazy,
            _ => return Err(()),
        })
    }
}

impl Mode {
    #[must_use]
    pub fn eager(b: bool) -> Self {
        if b {
            Self::Eager
        } else {
            Self::Lazy
        }
    }
    #[must_use]
    pub fn sigil(self) -> &'static str {
        match self {
            Self::Eager => EAGER_SIGIL,
            Self::Lazy => LAZY_SIGIL,
        }
    }
}

impl ToTokens for Mode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let sigil = match self {
            Self::Eager => EAGER_SIGIL,
            Self::Lazy => LAZY_SIGIL,
        };

        Ident::new(sigil, Span::call_site()).to_tokens(tokens);
    }
}

#[derive(Debug)]
pub struct Suspend;

impl TryFrom<&'_ str> for Suspend {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s == "suspend_eager" {
            Ok(Self)
        } else {
            Err(())
        }
    }
}

enum MacroType {
    Suspend(Suspend),
    ModeSwitch(Mode),
    Exec(ExecutableMacroType),
    Unknown,
}

impl MacroType {
    fn try_new(ty: MacroPathType, mode_only: bool) -> Option<Self> {
        #[derive(Debug)]
        enum IgnoreMode {
            DoNot,
            Prelude,
            Std,
            Core,
            Alloc,
        }

        let (macro_name, ignore, exec) = match (ty, mode_only) {
            (MacroPathType::CrateRootMacro { .. }, true) => return None,
            (
                MacroPathType::CrateRootMacro {
                    crate_name,
                    macro_name,
                    ..
                },
                false,
            ) => {
                let ignore = match crate_name.as_str() {
                    "std" => IgnoreMode::Std,
                    "core" => IgnoreMode::Core,
                    "alloc" => IgnoreMode::Alloc,
                    _ => return Some(Self::Unknown),
                };
                (macro_name, ignore, false)
            }
            (MacroPathType::UnpathedMacro { macro_name }, _) => {
                (macro_name, IgnoreMode::Prelude, true)
            }
            (
                MacroPathType::DollarCrateRootMacro { macro_name }
                | MacroPathType::Eager2CrateRootMacro { macro_name, .. }
                | MacroPathType::ReExportMacro { macro_name },
                _,
            ) => (macro_name, IgnoreMode::DoNot, true),
        };

        let macro_name = macro_name.as_str();

        // Try mode switches first
        if let Ok(suspend) = macro_name.try_into() {
            return Some(Self::Suspend(suspend));
        }
        if let Ok(mode) = macro_name.try_into() {
            return Some(Self::ModeSwitch(mode));
        }

        // Stop if we're limited to mode only
        if mode_only {
            return None;
        }

        // Try exec
        if exec {
            if let Ok(exec) = macro_name.try_into() {
                return Some(Self::Exec(exec));
            }
        }
        // Try ignore
        match (ignore, macro_name) {
            (
                IgnoreMode::Prelude | IgnoreMode::Std | IgnoreMode::Core,
                "assert" | "assert_eq" | "assert_ne" | "debug_assert" | "debug_assert_eq"
                | "debug_assert_ne" | "format_args" | "matches" | "panic" | "todo" | "try"
                | "unimplemented" | "unreachable" | "write" | "writeln",
            )
            | (
                IgnoreMode::Std | IgnoreMode::Core,
                "cfg" | "column" | "compile_error" | "concat" | "env" | "file" | "include"
                | "include_bytes" | "include_str" | "line" | "module_path" | "option_env"
                | "stringify",
            )
            | (
                IgnoreMode::Prelude | IgnoreMode::Std,
                "dbg"
                | "eprint"
                | "eprintln"
                | "is_x86_feature_detected"
                | "print"
                | "println"
                | "thread_local",
            )
            | (IgnoreMode::Prelude | IgnoreMode::Std | IgnoreMode::Alloc, "format" | "vec") => {
                return None;
            }
            _ => {}
        }

        Some(Self::Unknown)
    }
}

struct ModeSwitchTrailingMacro {
    offset: usize,
    mode: Mode,
}
struct ExecutableTrailingMacro {
    offset: usize,
    exec: ExecutableMacroType,
}
struct UnknownTrailingMacro {
    offset: usize,
}

enum TrailingMacro {
    ModeSwitch(ModeSwitchTrailingMacro),
    Exec(ExecutableTrailingMacro),
    Unknown(UnknownTrailingMacro),
}

#[derive(Debug)]
struct SuspendTrailingMacro {
    offset: usize,
}

impl TrailingMacro {
    fn try_new(
        tokens: &[TokenTree],
        mode_only: bool,
    ) -> Result<Option<Self>, SuspendTrailingMacro> {
        let Some(path) = MacroPathSegments::try_new(tokens) else {
            return Ok(None);
        };
        let offset = tokens.len() - path.tokens.len();

        Ok(Some(
            match path.ty.and_then(|ty| MacroType::try_new(ty, mode_only)) {
                None => return Ok(None),
                Some(MacroType::Suspend(_)) => return Err(SuspendTrailingMacro { offset }),
                Some(MacroType::ModeSwitch(mode)) => {
                    TrailingMacro::ModeSwitch(ModeSwitchTrailingMacro { offset, mode })
                }
                Some(MacroType::Exec(exec)) => {
                    TrailingMacro::Exec(ExecutableTrailingMacro { offset, exec })
                }
                Some(MacroType::Unknown) => TrailingMacro::Unknown(UnknownTrailingMacro { offset }),
            },
        ))
    }
    fn mode(&self) -> Option<Mode> {
        match self {
            TrailingMacro::ModeSwitch(m) => Some(m.mode),
            _ => None,
        }
    }
}
impl ModeSwitchTrailingMacro {
    fn truncate(self, tokens: &mut Vec<TokenTree>) -> Mode {
        tokens.truncate(self.offset);
        self.mode
    }
}
impl ExecutableTrailingMacro {
    fn truncate(self, tokens: &mut Vec<TokenTree>) -> ExecutableMacroType {
        tokens.truncate(self.offset);
        self.exec
    }
}
impl UnknownTrailingMacro {
    fn split_off(self, tokens: &mut Vec<TokenTree>) -> Vec<TokenTree> {
        tokens.split_off(self.offset)
    }
}
impl SuspendTrailingMacro {
    fn truncate(self, tokens: &mut Vec<TokenTree>) {
        tokens.truncate(self.offset);
    }
}
pub struct MacroPath {
    pub segments: Vec<TokenTree>,
}
impl ToTokens for MacroPath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for token in &self.segments {
            token.to_tokens(tokens);
        }
    }
}
enum Stack {
    Empty,
    Raw(Group),
    Processed(Box<State>, Option<TrailingMacro>),
}

impl Stack {
    fn take(&mut self) -> Self {
        mem::replace(self, Self::Empty)
    }
    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}
impl ToTokens for Stack {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Stack::Empty => {
                Group::new(Delimiter::Bracket, TokenStream::new()).to_tokens(tokens);
            }
            Stack::Raw(g) => g.to_tokens(tokens),
            Stack::Processed(s, _) => s.to_tokens(tokens),
        }
    }
}

pub struct State {
    span: Span,
    delim: Delimiter,
    mode: Mode,
    free: EfficientGroupT,
    locked: EfficientGroupT,
    processed: EfficientGroupV,
    stack: Stack,
    unprocessed: Vec<token_stream::IntoIter>,
}

impl ToTokens for State {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut state = TokenStream::new();
        self.mode.to_tokens(&mut state);
        self.free.to_tokens(&mut state);
        self.locked.to_tokens(&mut state);
        self.processed.to_tokens(&mut state);
        self.stack.to_tokens(&mut state);
        for stream in self.unprocessed.iter().rev() {
            state.extend(stream.clone());
        }

        Group::new(self.delim, state).to_tokens(tokens);
    }
}

pub enum ProcessingResult<'a> {
    Partial(MacroPath, &'a Literal, State, TokenStream),
    Done(FullyProccesedState),
}

impl ToTokens for ProcessingResult<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            // If we were eager and finished, just output the finished result
            Self::Done(processed) => processed.to_tokens(tokens),

            // If we were eager and didn't finish it means we found an external
            // eager macro we need to call, pass control over to them
            // and record our state
            Self::Partial(eager_macro, eager_call_sigil, state, stream) => {
                eager_macro.to_tokens(tokens);
                Group::new(
                    Delimiter::Brace,
                    eager_data(eager_call_sigil, state.to_token_stream(), stream.clone()),
                )
                .to_tokens(tokens);
            }
        }
    }
}

pub struct FullyProccesedState(State);

impl ToTokens for FullyProccesedState {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.free.append_to_stream(tokens);
        self.0.locked.append_to_stream(tokens);
        self.0.processed.append_to_stream(tokens);
    }
}
impl State {
    // Should only be called with the top level stream
    pub fn decode_from_stream<F>(
        stream: TokenStream,
        reprocess: bool,
        expansion: F,
    ) -> Result<Option<Self>, Error>
    where
        F: FnOnce(token_stream::IntoIter) -> Result<token_stream::IntoIter, Error>,
    {
        let span = Span::call_site();
        let mut stream = stream.into_iter();
        if expect_call_literal(stream.next_or(span)) {
            return Ok(None);
        }

        let group = expect_group(stream.next_or(span), Delimiter::Bracket)?;

        let span = group.span();
        let mut group = group.stream().into_iter();
        let state = expect_group(group.next_or(span), Param::Named("eager2_state"))?;
        if group.next().is_some() {
            return Err(Error {
                span,
                msg: "`eager2_state` should only contain one group".into(),
                note: None,
            });
        }

        Self::decode_from_group(state, Some((reprocess, expansion(stream)?))).map(Some)
    }

    // Only should be called on encoded groups
    #[allow(clippy::needless_pass_by_value)]
    fn decode_from_group(
        g: Group,
        extra: Option<(bool, token_stream::IntoIter)>,
    ) -> Result<Self, Error> {
        use Delimiter::Bracket;

        let span = g.span();
        let delim = g.delimiter();

        let mut stream = g.stream().into_iter();
        let mode = expect_mode(stream.next_or(span), "mode")?;
        let free = expect_group(stream.next_or(span), Bracket)?;
        let locked = expect_group(stream.next_or(span), Bracket)?;
        let processed = expect_group(stream.next_or(span), Bracket)?;
        let stack = expect_group(stream.next_or(span), Param::Named("stack"))?;

        let mut processed: EfficientGroupV = processed.into();
        let unprocessed = match extra {
            Some((true, extra)) => vec![stream, extra],
            Some((false, extra)) => {
                processed.extend(extra);
                vec![stream]
            }
            None => vec![stream],
        };

        Ok(Self {
            span,
            delim,
            mode,
            free: free.into(),
            locked: locked.into(),
            processed,
            stack: Stack::Raw(stack),
            unprocessed,
        })
    }

    #[must_use]
    pub fn new(span: Span, delim: Delimiter, mode: Mode, stream: TokenStream) -> Self {
        Self {
            span,
            delim,
            mode,
            free: EfficientGroupT::default(),
            locked: EfficientGroupT::default(),
            processed: EfficientGroupV::default(),
            stack: Stack::Empty,
            unprocessed: vec![stream.into_iter()],
        }
    }
    #[allow(clippy::result_large_err)]
    pub fn process(mut self, eager_call_sigil: &Literal) -> Result<ProcessingResult<'_>, Error> {
        while !self.unprocessed.is_empty() || !self.stack.is_empty() {
            self.process_tokens();
            if let Some((path, stream)) = self.pop_stack()? {
                return Ok(ProcessingResult::Partial(
                    path,
                    eager_call_sigil,
                    self,
                    stream,
                ));
            }
        }
        Ok(ProcessingResult::Done(FullyProccesedState(self)))
    }

    fn has_locking(&self) -> bool {
        !self.free.is_empty() || !self.locked.is_empty()
    }

    fn process_tokens(&mut self) {
        while let Some(tt) = self.unprocessed.pop_next() {
            let g = match tt {
                TokenTree::Group(g) => g,
                tt => {
                    self.processed.push(tt);
                    continue;
                }
            };

            // lookback to see if this group is a macro param
            // (looking especially for mode-switching macros)
            let tm = TrailingMacro::try_new(self.processed.as_mut_vec(), self.mode == Mode::Lazy);

            let tm = match tm {
                Ok(tm) => tm,
                Err(suspend) => {
                    suspend.truncate(self.processed.as_mut_vec());
                    let locked = EfficientGroupT::Processed(g.stream());
                    if self.has_locking() {
                        self.locked.append(self.processed.take());
                        self.locked.append(locked);
                    } else {
                        self.free.append(self.processed.take());
                        self.locked = locked;
                    }
                    continue;
                }
            };

            let inner_mode = tm
                .as_ref()
                .and_then(TrailingMacro::mode)
                .unwrap_or(self.mode);

            let stack = State::new(g.span(), g.delimiter(), inner_mode, g.stream());
            let stack = mem::replace(self, stack);
            self.stack = Stack::Processed(Box::new(stack), tm);
        }
    }

    fn pop_stack(&mut self) -> Result<Option<(MacroPath, TokenStream)>, Error> {
        use TrailingMacro::{Exec, ModeSwitch, Unknown};

        debug_assert!(self.unprocessed.is_empty());

        // `stack`` is the caller, `self` is the completed recursive call
        let (mut stack, tm) = match self.stack.take() {
            Stack::Empty => return Ok(None),
            Stack::Processed(p, tm) => (*p, tm),
            Stack::Raw(r) if r.stream().is_empty() => return Ok(None),
            Stack::Raw(r) => {
                let mut p = Self::decode_from_group(r, None)?;
                let tm = TrailingMacro::try_new(p.processed.as_mut_vec(), self.mode == Mode::Lazy);
                (p, tm.unwrap())
            }
        };

        // `self` is the caller, `stack` is the completed recursive call
        mem::swap(self, &mut stack);

        match (tm, self.mode) {
            // Handle mode switches
            (Some(ModeSwitch(tm)), _) => {
                match (
                    self.mode,
                    tm.truncate(self.processed.as_mut_vec()),
                    self.has_locking(),
                    stack.has_locking(),
                ) {
                    (_, Mode::Eager, _, false) => {
                        self.processed.append(stack.processed);
                    }
                    (_, Mode::Eager, false, true) => {
                        self.free.append(self.processed.take());
                        self.free.append(stack.free);
                        self.locked = stack.locked;
                        self.processed.append(stack.processed);
                    }
                    (_, Mode::Eager, true, true) => {
                        self.locked.append(self.processed.take());
                        self.locked.append(stack.locked);
                        self.processed.append(stack.processed);
                    }

                    (Mode::Eager, Mode::Lazy, false, _) => {
                        self.free.append(self.processed.take());
                        self.free.append(stack.free);
                        self.locked = stack.locked;
                        self.locked.append(stack.processed);
                    }
                    (Mode::Eager, Mode::Lazy, true, _) | (Mode::Lazy, Mode::Lazy, _, _) => {
                        self.locked.append(self.processed.take());
                        self.locked.append(stack.free);
                        self.locked.append(stack.locked);
                        self.locked.append(stack.processed);
                    }
                }
            }
            // Execute known macros
            (Some(Exec(tm)), Mode::Eager) => {
                let macro_type = tm.truncate(self.processed.as_mut_vec());
                let stream = stack
                    .free
                    .into_iter()
                    .chain(stack.locked)
                    .chain(stack.processed);

                macro_type.execute(
                    stack.span,
                    stream,
                    &mut self.processed,
                    &mut self.unprocessed,
                )?;
            }
            // Hand over execution to unknown macros if in eager mode
            (Some(Unknown(tm)), Mode::Eager) => {
                let segments = tm.split_off(self.processed.as_mut_vec());
                let path = MacroPath { segments };
                stack.free.append(stack.locked);
                stack.free.append(stack.processed);
                let stream = stack.free.into_stream();
                return Ok(Some((path, stream)));
            }

            (_, Mode::Lazy) | (None, Mode::Eager) => {
                stack.free.append(stack.locked);
                stack.free.append(stack.processed);
                let stream = stack.free.into_stream();
                let group = Group::new(stack.delim, stream);

                self.processed.push(TokenTree::Group(group));
            }
        }

        Ok(None)
    }
}
