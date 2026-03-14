
# LeetCode 19: Remove Nth Node From End Of List
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

def removeNthFromEnd(head: ListNode, n: int) -> ListNode:
    dummy = ListNode(0, head)
    left = dummy
    right = head

    while n > 0:
        right = right.next
        n -= 1

    while right:
        left = left.next
        right = right.next

    # delete
    left.next = left.next.next
    return dummy.next



def main():
    assert list_node_to_string(removeNthFromEnd(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None))))), 2)) == list_node_to_string(ListNode(1, ListNode(2, ListNode(3, ListNode(5, None)))))
    assert removeNthFromEnd(ListNode(1, None), 1) == None
    assert list_node_to_string(removeNthFromEnd(ListNode(1, ListNode(2, None)), 1)) == list_node_to_string(ListNode(1, None))

if __name__ == "__main__":
    main()