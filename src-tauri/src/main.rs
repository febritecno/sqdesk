// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// The debug binary's unwind tables exceed the 16MB that macOS `ld` can index
// in its compact unwind table, so it notes that it fell back to plain DWARF
// unwinding. That only makes a panic's stack walk slower in debug builds; it
// is not a defect in the binary. `unknown_lints` keeps toolchains that predate
// the `linker_messages` lint quiet.
#![allow(unknown_lints, linker_messages)]

fn main() {
    sqdesk_lib::run();
}
