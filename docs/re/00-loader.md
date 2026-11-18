# Loading MITTELL.EXE for analysis

`MITTELL.EXE` (796945 bytes) is an MZ stub plus an LE image for DOS/4GW, built with Watcom C (entry jumps over a `WATCOM` signature). It has two objects and 13383 internal fixups, all of them object-relative (no imports).

| object | linear base | vsize | file pages | contents |
|---|---|---|---|---|
| 1 | `0x10000` | `0x61379` | 1-98 | code |
| 2 | `0x80000` | `0x71590` | 99-165 (~`0x43000` bytes; the rest is BSS) | data, strings, tables |

Entry point: linear `0x6B098`. Initial ESP: object 2 + `0x71590`.

## Reproduce

1. `python3 -m venv v && v/bin/pip install capstone` (only needed for disassembly).
2. Extract with fixups applied:
   `python le_extract.py MITTELL.EXE OUTDIR`
   The script lives outside the repo at `<scratch>`. It writes:
   - `obj1_00010000.bin`, `obj2_00080000.bin`: each object with fixups applied;
   - `image.bin`: both objects in one flat file starting at linear `0x10000` (file offset = linear − `0x10000`);
   - `layout.txt`: the table above.
   The method: read the LE header at `[0x3C]`. The object page map gives a 24-bit big-endian page number per page, and the file offset is `data_pages + (n−1)·page_size`. Each fixup record (source type 7 = 32-bit offset, 8 = 32-bit relative, 2 = selector, 5 = 16-bit offset, 6 = 16:32) is resolved to `object_base[target] + offset`.
3. Ghidra (12.2): import `image.bin` with the raw binary loader at base `0x10000`, language `x86:LE:32:default`:
   `analyzeHeadless <projdir> <name> -import image.bin -loader BinaryLoader -loader-baseAddr 0x10000 -processor x86:LE:32:default`
   Watcom passes arguments in EAX, EDX, EBX, ECX, so the decompiler hides most call arguments. Read the call site disassembly to get them.

In Rust, `mad_assets::le::LeExe` maps linear addresses to file bytes without applying fixups. That is enough for the palette and the strings.

## Useful addresses (data object)

| linear | what |
|---|---|
| `0x8383D` | 256×3 VGA palette (6-bit), uploaded by the graphics init via INT 10h AX=1012h |
| `0x84D74` / `0x84D86` | `icons?\icon00.dat` / `icons?b\icon??.dat` name templates |
| `0x80450` | `smpxxkhz\smpl_xxx.dat` template |
| `0x8692E` / `0x86940` | `confi2\config.dat` / `confi2\game0.dat` |
| `0x86954` | CONFIG.DAT image (0x279 bytes) |
| `0xBB794` | `szenario.dat`; loaded to `0xD9D00` |
| `0xC31A0`, `0xC27A0`, `0xC2DA0`, `0xC29A0` | ICONIN.DAT tables (size, flags, width, height) |

Function addresses (code): icon loader `0x2DD70`, icon system init (reads ICONIN and ICONCS) `0x2F4B0`, CONFIG load/save `0x42030`/`0x421A0`, game load/save `0x42200`/`0x42360`, SZENARIO load `0x3E2A0`, battle deployment from SZENARIO `0x3DDA0`.
