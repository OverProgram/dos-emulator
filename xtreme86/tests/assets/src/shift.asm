CPU 286

    mov cx, 0xFFFE
    ror cx, 1
    nop
    rol cx, 1
    nop

    mov dx, 0xFE00
    stc
    rcr dh, 1
    nop

    clc
    rcl dh, 1
    nop

    mov bx, 0x00FF
    sal bl, 4
    nop

    shr bl, 4
    nop

    mov si, 0x800F
    sar si, 2
    nop
