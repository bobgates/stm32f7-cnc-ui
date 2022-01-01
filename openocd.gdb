target extended-remote :3333

set print asm-demangle on

break DeafaultHandler
break HardFault
break rust_begin_unwind
monitor arm semihosting enable

load
continue
