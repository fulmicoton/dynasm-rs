#![feature(plugin)]
#![plugin(dynasm)]
extern crate dynasmrt;
use dynasmrt::DynAsmApi;

macro_rules! test {
    () => (mov rax, rbx)
}

fn main() {
    let mut ops = dynasmrt::Assembler::new();
    let d = 3;
    let c = 4;

    // interesting testcases
    dynasm!(ops
        // no args
        ; ret
        // immediate
        ; ret 16
        // register
        ; inc rax
        // memory ref
        ; inc DWORD [16]
        ; inc DWORD [rax]
        ; inc DWORD [rax*2]
        ; inc DWORD [rax*3]
        ; inc DWORD [rax*4]
        ; inc DWORD [rax*5]
        ; inc DWORD [rax*8]
        ; inc DWORD [rax*9]
        ; inc DWORD [rax + 16]
        ; inc DWORD [rax*8 + 16]
        ; inc DWORD [rax + rbx]
        ; inc DWORD [rax + rbx + 16]
        ; inc DWORD [rax*8 + rbx + 16]
        // special memoryref cases
        ; inc DWORD [rsp]
        ; inc DWORD [r12]
        ; inc DWORD [rsp + rax]
        ; inc DWORD [rax + rsp]
        ; inc DWORD [rbp]
        ; inc DWORD [r13]
        ; inc DWORD [rbp + 16]
        ; inc DWORD [rbp*8]
        ; inc DWORD [rip]
        ; inc DWORD [rip + 16]
        // weird registers
        ; xchg al, ah
        ; xchg al, dil
        // register-specific forms
        ; adc rax, 5
        // multi arg forms
        ; mov rax, rbx
        ; mov rax, [rbx]
        ; mov [rbx], rax
        ; mov rax, 1
        ; mov [rax], BYTE 1
        ; imul rax, rbx, 1
        ; imul rax, [rbx], 1
        // prefixes
        ; fs inc DWORD [rax]
        ; lock fs inc DWORD [rax]
        ; rep stosq
        ; inc DWORD [eax]
        // really long instructions
        ; fs imul r9w, [r10d*8 + r11d + 0x66778899], 0x1122
        ; fs imul r9,  [edi*8 + r11d + 0x66778899], 0x11223344
        ; fs mov r9, QWORD 0x1122334455667788
        ; fs movabs rax, 0x1122334455667788
        // funky syntax features
        ; inc BYTE [rax]
        ; inc WORD [rax]
        ; inc DWORD [rax]
        ; inc QWORD [rax]
        // very odd memoryrefs
        ; mov rax, [rbx + rbx * 3 + 2 + c + rax + d]
        // labels
        ; a: // local
        ; -> b: // global
        ; => 1 // dynamic. note the lack of a trailing :. this is due to : being a valid symbol within expressions that does not occur in any other normal rust expr contexts.
        // jumps
        ; jmp <a
        ; jmp -> b
        ; jmp => 1
        // rip relative stuff
        ; lea rax, [->b]
        // dynamic registers
        ; inc Rb(1)
        ; inc Rh(5)
        ; inc Rw(1)
        ; inc Rd(1)
        ; inc Rq(1)
        ; mov Rb(7), [Rq(3)*4 + rax]
        ; fsub Rf(5), st0
        // other register families
        ; mov cr1, rax
        ; mov dr1, rax
        ; mov rax, cr1
        ; mov rax, dr1
        ; pop fs
        ; movmskps eax, xmm7
        ; movd mmx7, eax
        ; movd eax, mmx7
        ; fcomp st0
        // VEX/XOP instructions
        ; andn rax, rcx, rdx
        ; andn r8, r9, r10
        ; bextr rax, rbx, 1
        ; vaddpd xmm0, xmm1, [rax]
        // VSIB addressing
        ; vgatherqpd ymm1, QWORD [ymm15*8 + rsi + 0x11112222], ymm8
        // 4 argument instructions
        ; vfmaddss xmm0, xmm1, xmm2, xmm3
        // directives
        ; string:
        ; .bytes "Hello world!\0".bytes()
    );

    let index = ops.offset();
    dynasm!(ops
        ; mov eax, 10203040
        ; ret
    );

    let buf = ops.finalize().unwrap();

    println!("Generated assembly:");
    for i in buf.iter() {
        print!("{:02x }", i);
    }
    println!("");

    let func: extern "C" fn() -> i64 = unsafe { std::mem::transmute(buf.ptr(index)) };
    println!("assembled function result: {}", func() );
}
