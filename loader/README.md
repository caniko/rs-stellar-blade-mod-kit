# Loader Notes

The runtime DLL is intended to be loaded by a UE4SS-compatible Bouldy V2 shim. The shim must provide:

- logging;
- delta seconds;
- tick and shutdown registration;
- a generic discovery backend that scans reflected Unreal objects, classes, functions, and properties;
- an export callback for structured reconnaissance records.

The Bouldy sample shim in `rs_bouldy/cpp-shim` shows the ABI shape, but its discovery function is a stub. A real Stellar Blade deployment needs a UE4SS-backed implementation.
