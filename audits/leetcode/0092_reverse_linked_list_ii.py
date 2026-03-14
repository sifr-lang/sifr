# LeetCode 92: Reverse Linked List Ii
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

def reverseBetween(
    self, head: Optional[ListNode], left: int, right: int
) -> Optional[ListNode]:
    dummy = ListNode(0, head)

    # 1) reach node at position "left"
    leftPrev, cur = dummy, head
    for i in range(left - 1):
        leftPrev, cur = cur, cur.next

    # Now cur="left", leftPrev="node before left"
    # 2) reverse from left to right
    prev = None
    for i in range(right - left + 1):
        tmpNext = cur.next
        cur.next = prev
        prev, cur = cur, tmpNext

    # 3) Update pointers
    leftPrev.next.next = cur  # cur is node after "right"
    leftPrev.next = prev  # prev is "right"
    return dummy.next



def main():
    assert list_node_to_string(
        reverseBetween(
            None,
            ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5))))),
            2,
            4,
        )
    ) == "1->4->3->2->5"
    assert list_node_to_string(reverseBetween(None, ListNode(5), 1, 1)) == "5"

if __name__ == "__main__":
    main()
