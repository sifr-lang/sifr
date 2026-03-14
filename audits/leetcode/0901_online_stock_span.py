
# LeetCode 901: Online Stock Span
# Python version

class StockSpanner:
    def __init__(self):
        self.stack = []  # pair: (price, span)
    def next(self, price: int) -> int:
        span = 1
        while self.stack and self.stack[-1][0] <= price:
            span += self.stack[-1][1]
            self.stack.pop()
        self.stack.append((price, span))
        return span

def main():
    obj = StockSpanner()
    assert obj.next(100) == 1
    assert obj.next(80) == 1
    assert obj.next(60) == 1
    assert obj.next(70) == 2
    assert obj.next(60) == 1
    assert obj.next(75) == 4
    assert obj.next(85) == 6

if __name__ == "__main__":
    main()
