# LeetCode 1472: Design Browser History
# Python version

class Node:
    def __init__(
        self,
        val: int = 0,
        next: 'Node | None' = None,
        random: 'Node | None' = None,
        left: 'Node | None' = None,
        right: 'Node | None' = None,
        neighbors: list['Node'] | None = None,
        key: int = -1,
    ):
        self.val = val
        self.next = next
        self.random = random
        self.left = left
        self.right = right
        self.neighbors = [] if neighbors is None else neighbors
        self.key = key

class ListNode:
    def __init__(self, val, prev=None, next=None):
        self.val = val
        self.prev = prev
        self.next = next
class BrowserHistory:
    def __init__(self, homepage: str):
        self.cur = ListNode(homepage)
    def visit(self, url: str) -> None:
        self.cur.next = ListNode(url, self.cur)
        self.cur = self.cur.next
    def back(self, steps: int) -> str:
        while self.cur.prev and steps > 0:
            self.cur = self.cur.prev
            steps -= 1
        return self.cur.val
    def forward(self, steps: int) -> str:
        while self.cur.next and steps > 0:
            self.cur = self.cur.next
            steps -= 1
        return self.cur.val
class BrowserHistory:    
    def __init__(self, homepage: str):
        self.i = 0
        self.len = 1
        self.history = [homepage]
    def visit(self, url: str) -> None:
        if len(self.history) < self.i + 2:
            self.history.append(url)
        else:
            self.history[self.i + 1] = url
        self.i += 1
        self.len = self.i + 1
    def back(self, steps: int) -> str:
        self.i = max(self.i - steps, 0)
        return self.history[self.i]
    def forward(self, steps: int) -> str:
        self.i = min(self.i + steps, self.len - 1)
        return self.history[self.i]

def main():
    obj = BrowserHistory('leetcode.com')
    obj.visit('google.com')
    obj.visit('facebook.com')
    obj.visit('youtube.com')
    assert obj.back(1) == 'facebook.com'
    assert obj.back(1) == 'google.com'
    assert obj.forward(1) == 'facebook.com'
    obj.visit('linkedin.com')
    assert obj.forward(2) == 'linkedin.com'
    assert obj.back(2) == 'google.com'
    assert obj.back(7) == 'leetcode.com'

if __name__ == "__main__":
    main()