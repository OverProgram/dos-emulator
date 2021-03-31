    mov ax, 0x00
    call proc
    nop

proc:
    enter 5, 0
    mov ax, 0x16
    leave
    ret
