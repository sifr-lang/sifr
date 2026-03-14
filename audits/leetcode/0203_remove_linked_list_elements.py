# LeetCode 203: Remove Linked List Elements
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

def removeElements(head: ListNode, val: int) -> ListNode:
    dummy = ListNode(next=head)
    prev, curr = dummy, head
    
    while curr:
        nxt = curr.next
        
        if curr.val == val:
            prev.next = nxt
        else:
            prev = curr
        
        curr = nxt
    return dummy.next



def main():
    assert list_node_to_string(removeElements(ListNode(1, ListNode(2, ListNode(6, ListNode(3, ListNode(4, ListNode(5, ListNode(6, None))))))), 6)) == list_node_to_string(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None))))))
    assert removeElements(None, 1) == None
    assert removeElements(ListNode(7, ListNode(7, ListNode(7, ListNode(7, None)))), 7) == None

if __name__ == "__main__":
    main()