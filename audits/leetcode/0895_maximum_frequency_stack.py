# LeetCode 895: Maximum Frequency Stack
# Python version

class FreqStack:
    def __init__(self):
        self.cnt = {}
        self.maxCnt = 0
        self.stacks = {}
    def push(self, val: int) -> None:
        valCnt = 1 + self.cnt.get(val, 0)
        self.cnt[val] = valCnt
        if valCnt > self.maxCnt:
            self.maxCnt = valCnt
            self.stacks[valCnt] = []
        self.stacks[valCnt].append(val)
    def pop(self) -> int:
        res = self.stacks[self.maxCnt].pop()
        self.cnt[res] -= 1
        if not self.stacks[self.maxCnt]:
            self.maxCnt -= 1
        return res

def main():
    obj = FreqStack()
    obj.push(5)
    obj.push(7)
    obj.push(5)
    obj.push(7)
    obj.push(4)
    obj.push(5)
    assert obj.pop() == 5
    assert obj.pop() == 7
    assert obj.pop() == 5
    assert obj.pop() == 4

if __name__ == "__main__":
    main()
