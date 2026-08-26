import torch


def run() -> str:
    tensor = torch.tensor([1.0, 2.0, 3.0, 4.0, 5.0, 6.0], dtype=torch.float32)
    doubled = torch.mul(tensor.reshape(2, 3), 2.0)
    criterion = torch.nn.LinearCrossEntropyLoss(3, 2)
    with torch.no_grad():
        criterion.linear.weight.copy_(torch.tensor([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]))
    loss = criterion(
        torch.tensor([[2.0, 1.0, 0.0]], dtype=torch.float32),
        torch.tensor([0], dtype=torch.int64),
    )
    values = doubled.flatten().tolist()
    if values != [2.0, 4.0, 6.0, 8.0, 10.0, 12.0]:
        raise RuntimeError("Torch values did not round-trip")
    if (
        float(doubled.sum().item()) != 42.0
        or list(doubled.shape) != [2, 3]
        or doubled.dtype != torch.float32
        or abs(float(loss.item()) - 0.31326166) >= 1e-6
    ):
        raise RuntimeError("Torch metadata did not match")
    return "sifr-python-interop:torch:sum=42.0:shape=2x3:dtype=float32:linear-xent=ok"
