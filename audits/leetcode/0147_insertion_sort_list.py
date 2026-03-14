# LeetCode 147: Insertion Sort List
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

def insertionSortList(head: Optional[ListNode]) -> Optional[ListNode]:
    if not head or not head.next:
        return head

    sentinel = ListNode()
    curr = head
    while curr:
        prev = sentinel
        while prev.next and curr.val >= prev.next.val:
            prev = prev.next

        curr.next, prev.next, curr = prev.next, curr, curr.next

    return sentinel.next



def main():
    assert list_node_to_string(insertionSortList(ListNode(4, ListNode(2, ListNode(1, ListNode(3, None)))))) == list_node_to_string(ListNode(1, ListNode(2, ListNode(3, ListNode(4, None)))))
    assert list_node_to_string(insertionSortList(ListNode(-1, ListNode(5, ListNode(3, ListNode(4, ListNode(0, None))))))) == list_node_to_string(ListNode(-1, ListNode(0, ListNode(3, ListNode(4, ListNode(5, None))))))

if __name__ == "__main__":
    main()