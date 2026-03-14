# LeetCode 83: Remove Duplicates From Sorted List
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

def deleteDuplicates(head: Optional[ListNode]) -> Optional[ListNode]:
    cur = head
    while cur:
        while cur.next and cur.next.val == cur.val:
            cur.next = cur.next.next
        cur = cur.next
    return head



def main():
    assert list_node_to_string(deleteDuplicates(ListNode(1, ListNode(1, ListNode(2, None))))) == list_node_to_string(ListNode(1, ListNode(2, None)))
    assert list_node_to_string(deleteDuplicates(ListNode(1, ListNode(1, ListNode(2, ListNode(3, ListNode(3, None))))))) == list_node_to_string(ListNode(1, ListNode(2, ListNode(3, None))))

if __name__ == "__main__":
    main()