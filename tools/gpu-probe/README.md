# gpu-probe

Times an ONNX model on the CPU and on a GPU through ONNX Runtime, to see whether a GPU is worth using for a plugin on this machine.

```
cargo build --release
./target/release/gpu-probe <model.onnx> <input shape, like 1,3,640,640> <cpu|dml:N> <runs>
```

`dml:N` is DirectML (Windows, any DirectX 12 GPU) on adapter number N; try 0, 1... because an integrated GPU may be one of them and be slower than the CPU. The model must take one input of numbers of the given shape (the first input is used; the first dimension is the batch). The first three runs are not counted. A DirectML build puts `DirectML.dll` next to the program. Results and what they mean: `docs/research/gpu.md` in the Loomas repository.
