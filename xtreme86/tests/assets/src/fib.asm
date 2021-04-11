CPU 286

    mov cx, 10
    mov bx, 0
    mov word [0], 0
    mov word [2], 1
fib:
    nop
    mov ax, [bx]
    add bx, 2
    add ax, [bx]
    mov word [bx+2], ax
    loop fib
