# LeetCode 2130: Maximum Twin Sum Of A Linked List
# Python version

class ListNode:
    def __init__(self, val: int = 0, next: 'ListNode | None' = None):
        self.val = val
        self.next = next


def list_node_to_string(node: ListNode | None) -> str:
    parts = []
    cur = node
    while cur is not None:
        parts.append(str(cur.val))
        cur = cur.next
    return "->".join(parts) if parts else "None"


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

def pairSum(head: Optional[ListNode]) -> int:
    slow, fast = head, head
    prev = None
    while fast and fast.next:
        fast = fast.next.next
        tmp = slow.next
        slow.next = prev
        prev = slow
        slow = tmp

    res = 0
    while slow:
        res = max(res, prev.val + slow.val)
        prev = prev.next
        slow = slow.next
    return res



def main():
    assert pairSum(ListNode(5, ListNode(4, ListNode(2, ListNode(1, None))))) == 6
    assert pairSum(ListNode(4, ListNode(2, ListNode(2, ListNode(3, None))))) == 7
    assert pairSum(ListNode(1, ListNode(100000, None))) == 100001

if __name__ == "__main__":
    main()