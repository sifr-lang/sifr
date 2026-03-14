# LeetCode 61: Rotate List
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

def rotateRight(head: Optional[ListNode], k: int) -> Optional[ListNode]:
    if not head or not head.next or k == 0:
        return head

    old_head = head

    curr, size = head, 0
    while curr:
        curr, size = curr.next, size + 1

    if k % size == 0:
        return head

    k %= size
    slow = fast = head
    while fast and fast.next:
        if k <= 0:
            slow = slow.next
        fast = fast.next
        k -= 1

    new_tail, new_head, old_tail = slow, slow.next, fast
    new_tail.next, old_tail.next = None, old_head

    return new_head



def main():
    assert list_node_to_string(rotateRight(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None))))), 2)) == list_node_to_string(ListNode(4, ListNode(5, ListNode(1, ListNode(2, ListNode(3, None))))))
    assert list_node_to_string(rotateRight(ListNode(0, ListNode(1, ListNode(2, None))), 4)) == list_node_to_string(ListNode(2, ListNode(0, ListNode(1, None))))

if __name__ == "__main__":
    main()