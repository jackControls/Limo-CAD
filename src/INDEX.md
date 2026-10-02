# src/ (UI) index

| Path | Role |
|------|------|
| [main.tsx](main.tsx) | Browser React bootstrap |
| [store/](store/) | App state (mode, tool, document, solids) |
| [engine/](engine/) | Browser WASM engine API (TS) |
| [cam/](cam/) | CAM document mutations, setup defaults, and program export |
| [components/cam/](components/cam/) | Manufacturing browser, inspectors, and interactive 3D stock/toolpath simulation preview |
| [files/projectFiles.ts](files/projectFiles.ts) | Project open/save/import/export and crash recovery |
| [files/projectTabs.ts](files/projectTabs.ts) | Multi-document tab sessions over one hydrated browser OCCT engine |
