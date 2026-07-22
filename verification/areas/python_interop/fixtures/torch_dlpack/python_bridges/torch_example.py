import torch


def run() -> str:
    tensor = torch.tensor([1.0, 2.0, 3.0, 4.0, 5.0, 6.0], dtype=torch.float32)
    doubled = torch.mul(tensor.reshape(2, 3), 2.0)
    values = doubled.flatten().tolist()
    if values != [2.0, 4.0, 6.0, 8.0, 10.0, 12.0]:
        raise RuntimeError("Torch values did not round-trip")
    if (
        float(doubled.sum().item()) != 42.0
        or list(doubled.shape) != [2, 3]
        or doubled.dtype != torch.float32
    ):
        raise RuntimeError("Torch metadata did not match")
    return "sifr-python-interop:torch:sum=42.0:shape=2x3:dtype=float32"
