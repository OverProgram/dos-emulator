RM = rm

ASM = nasm
ASM_FLAGS = -f bin

SRC_DIR = src
OBJ_DIR = ../obj

_SRC = alu.asm cmp.asm fib.asm int.asm jmp.asm jmp_cond.asm lea.asm proc.asm str.asm flag.asm load.asm ex.asm far.asm shift.asm
SRC = $(_SRC:%=$(SRC_DIR)/%)

OBJ = $(_SRC:%.asm=$(OBJ_DIR)/%.out)

$(OBJ_DIR)/%.out:%.asm
	$(ASM) $(ASM_FLAGS) $< -o $@

all:$(OBJ)

clean:
	$(RM) $(OBJ)
