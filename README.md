# rgb
A Game Boy emulator in Rust.

## Mappers
| Mapper        | Supported          |
| ------------- | ------------------ |
| NONE          | :heavy_check_mark: |
| MBC1          | :heavy_check_mark: |
| MBC2          | :x:                |
| MBC3          | :x:                |
| MBC5          | :x:                |
| MBC6          | :x:                |
| MBC7          | :x:                |
| Pocket Camera | :x:                |
| TAMA5         | :x:                |
| HuC-1         | :x:                |
| HuC-3         | :x:                |
| MMM01         | :x:                |

## Tests
### cpu_instrs
| Test                  | Result             |
| --------------------- | ------------------ |
| 01-special            | :heavy_check_mark: |
| 02-interrupts         | :heavy_check_mark: |
| 03-op sp,hl           | :heavy_check_mark: |
| 04-op r,imm           | :heavy_check_mark: |
| 05-op rp              | :heavy_check_mark: |
| 06-ld r,r             | :heavy_check_mark: |
| 07-jr,jp,call,ret,rst | :heavy_check_mark: |
| 08-misc instrs        | :heavy_check_mark: |
| 09-op r,r             | :heavy_check_mark: |
| 10-bit ops            | :heavy_check_mark: |
| 11-op a,(hl)          | :heavy_check_mark: |

### instr_timing
| Test                  | Result             |
| --------------------- | ------------------ |
| instr_timing          | :heavy_check_mark: |