CPU 286

    mov ax, 0x00
    call proc
    nop
    call proc2
    nop

proc:
    enter 5, 0
    mov ax, 0x16
    leave
    ret

proc2:
    pusha
    popa
    ret
