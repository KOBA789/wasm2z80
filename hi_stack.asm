  CALL entry
  HALT
entry:
  ; i32.const 0
  LD DE,0
  PUSH DE ; lower 16 bits first
  LD DE,0
  PUSH DE
  ; i32.const 104
  LD DE,104
  PUSH DE
  LD DE,0
  PUSH DE
  ; i32.store8 offset=65535
  POP DE
  POP DE
  POP IX
  POP IX
  LD BC,65535
  ADD IX,BC
  LD (IX+0),E
  RET
