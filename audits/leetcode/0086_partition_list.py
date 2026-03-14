
# LeetCode 86: Partition List
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

def partition(head: ListNode | None, x: int) -> ListNode | None:
    less_head, bigger_head = ListNode(-1), ListNode(-1)
    less_prev, bigger_prev = less_head, bigger_head
    while head:
        if head.val < x:
            less_prev.next = head
            less_prev = less_prev.next
        else:
            bigger_prev.next = head
            bigger_prev = bigger_prev.next

        head = head.next

    less_prev.next = bigger_prev.next = None
    less_prev.next = bigger_head.next

    return less_head.next



def main():
    assert list_node_to_string(partition(ListNode(1, ListNode(4, ListNode(3, ListNode(2, ListNode(5, ListNode(2, None)))))), 3)) == list_node_to_string(ListNode(1, ListNode(2, ListNode(2, ListNode(4, ListNode(3, ListNode(5, None)))))))
    assert list_node_to_string(partition(ListNode(2, ListNode(1, None)), 2)) == list_node_to_string(ListNode(1, ListNode(2, None)))

if __name__ == "__main__":
    main()