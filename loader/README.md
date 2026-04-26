# Loader Notes

The runtime DLL is intended to be loaded by a UE4SS-compatible Bouldy V3 shim. The shim must provide:

- logging;
- delta seconds;
- tick and shutdown registration;
- a generic discovery backend that scans reflected Unreal objects, classes, functions, and properties;
- an export callback for structured reconnaissance records.
- SQLite persistence for scan runs, raw candidates, and exported combat records.

`ue4ss-cpp/` contains the adapter core, SQLite persistence layer, and UE4SS integration seam. The local tests use fake candidates so the database and filtering behavior can be validated without UE4SS headers.
