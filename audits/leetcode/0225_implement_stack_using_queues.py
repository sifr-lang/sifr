# LeetCode 225: Implement Stack Using Queues
# Python version

class MyStack:
    def __init__(self):
        self.q = deque()
    def push(self, x: int) -> None:
        self.q.append(x)
        for _ in range(len(self.q) - 1):
            self.q.append(self.q.popleft())
    def pop(self) -> int:
        return self.q.popleft()
    def top(self) -> int:
        return self.q[0]
    def empty(self) -> bool:
        return len(self.q) == 0

def main():
    obj = MyStack()
    obj.push(1)
    obj.push(2)
    assert obj.top() == 2
    assert obj.pop() == 2
    assert obj.empty() == False

if __name__ == "__main__":
    main()
