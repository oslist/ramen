    BOTPAK   EQU     0x00501000          ; bootpackのロード先

    ; BOOT_INFO関係
    CYLS     EQU     0x0ff0              ; ブートセクタが設定する
    LEDS     EQU     0x0ff1

    ORG      0xc200

    %include "vbe.asm"

    ; キーボードのLED状態をBIOSに教えてもらう

    MOV      AH,0x02
    INT      0x16                        ; keyboard BIOS
    MOV      [LEDS],AL

    ; PICが一切の割り込みを受け付けないようにする
    ; AT互換機の仕様では、PICの初期化をするなら、
    ; こいつをCLI前にやっておかないと、たまにハングアップする
    ; PICの初期化はあとでやる

    MOV      AL,0xff
    OUT      0x21,AL
    NOP                                  ; OUT命令を連続させるとうまくいかない機種があるらしいので
    OUT      0xa1,AL

    CLI                                  ; さらにCPUレベルでも割り込み禁止

    ; CPUから1MB以上のメモリにアクセスできるように、A20GATEを設定

    CALL     waitkbdout
    MOV      AL,0xd1
    OUT      0x64,AL
    CALL     waitkbdout
    MOV      AL,0xdf                     ; enable A20
    OUT      0x60,AL
    CALL     waitkbdout

    ; プロテクトモード移行

    LGDT     [GDTR0]                     ; 暫定GDTを設定
    MOV      EAX,CR0
    AND      EAX,0x7fffffff              ; bit31を0にする（ページング禁止のため）
    OR       EAX,0x00000001              ; bit0を1にする（プロテクトモード移行のため）
    MOV      CR0,EAX
    JMP      CODE_SEGMENT:pipelineflush

    [BITS 32]
pipelineflush:
    MOV      AX,DATA_SEGMENT                      ; 読み書き可能セグメント32bit
    MOV      DS,AX
    MOV      ES,AX
    MOV      FS,AX
    MOV      GS,AX
    MOV      SS,AX

    ; bootpackの転送

    MOV      ESI,bootpack                ; 転送元
    MOV      EDI,BOTPAK                  ; 転送先
    MOV      ECX,512*1024/4
    CALL     memcpy

    ; asmheadでしなければいけないことは全部し終わったので、
    ; あとはbootpackに任せる

    ; bootpackの起動

    %include "paging.asm"

    MOV      ESP,0xC0080FFF                ; スタック初期値
    JMP      0xC0000000

waitkbdout:
    IN       AL,0x64
    AND      AL,0x02
    JNZ      waitkbdout                  ; ANDの結果が0でなければwaitkbdoutへ
    RET

memcpy:
    MOV      EAX,[ESI]
    ADD      ESI,4
    MOV      [EDI],EAX
    ADD      EDI,4
    SUB      ECX,1
    JNZ      memcpy                      ; 引き算した結果が0でなければmemcpyへ
    RET
    ; memcpyはアドレスサイズプリフィクスを入れ忘れなければ、ストリング命令でも書ける

    ALIGNB   16, DB 0
GDT0:
    TIMES    8 DB 0                      ; ヌルセレクタ
    DATA_SEGMENT    EQU 0x08
    DW       0xffff,0x0000,0x9200,0x00cf ; 読み書き可能セグメント32bit
    CODE_SEGMENT    EQU 0x10
    DW       0xffff,0x0000,0x9a00,0x00cf ; Executable 32bit

    DW       0
GDTR0:
    DW       8*4-1
    DD       GDT0

    ALIGNB   16
bootpack:
