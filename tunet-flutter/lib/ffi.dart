// This file initializes the dynamic library and connects it with the stub
// generated by flutter_rust_bridge_codegen.

// Re-export the bridge so it is only necessary to import this file.
export 'frb_generated.dart';

// Re-export all of your API files here
export 'api.dart';

// Re-export necessary types.
export 'lib.dart';
