# eager2-core

This is a helper crate for [`eager2`](../README.md). It exists to reduce compile times by:
* Having 0 dependencies (except for those used for testing)
* Not having a build script
* Containing as much of the code as possible (i.e. reducing the `eager2` code which needs to be compiled)
