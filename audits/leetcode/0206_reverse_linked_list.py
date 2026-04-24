
# LeetCode 206: Reverse Linked List
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


def reverseList(head: ListNode | None) -> ListNode | None:
    prev, curr = None, head

    while curr:
        temp = curr.next
        curr.next = prev
        prev = curr
        curr = temp
    return prev



def main():
    assert list_node_to_string(reverseList(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None))))))) == list_node_to_string(ListNode(5, ListNode(4, ListNode(3, ListNode(2, ListNode(1, None))))))
    assert list_node_to_string(reverseList(ListNode(1, ListNode(2, None)))) == list_node_to_string(ListNode(2, ListNode(1, None)))
    assert reverseList(None) == None

if __name__ == "__main__":
    main()
