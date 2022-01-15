use std::time::Duration;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use gband::{borrow_cpu_bus, Emulator};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cpu");
    group.warm_up_time(Duration::from_millis(500));
    group.sample_size(100);
    group.measurement_time(Duration::from_millis(500));

    for opcode in [0x39, 0x40, 0x41, 0x50, 0x5E, 0x66, 0x7F, 0x80] {
        group.bench_with_input(BenchmarkId::new("fetch", opcode), &opcode, |b, opcode| {
            let rom = vec![*opcode, 69];
            let mut emulator = Emulator::new(&rom, None).unwrap();
            b.iter(|| {
                let mut cpu_bus = borrow_cpu_bus!(emulator);
                emulator.cpu.fetch(&mut cpu_bus);
            })
        });

        group.bench_with_input(BenchmarkId::new("execute", opcode), &opcode, |b, opcode| {
            let rom = vec![*opcode, 69];
            let mut emulator = Emulator::new(&rom, None).unwrap();
            b.iter(|| {
                let mut cpu_bus = borrow_cpu_bus!(emulator);
                emulator.cpu.execute(&mut cpu_bus);
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
