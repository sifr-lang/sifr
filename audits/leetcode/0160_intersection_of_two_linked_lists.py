
# LeetCode 160: Intersection Of Two Linked Lists
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
def getIntersectionNode(
    self, headA: ListNode, headB: ListNode
) -> ListNode | None:
    l1, l2 = headA, headB
    while l1 != l2:
        l1 = l1.next if l1 else headB
        l2 = l2.next if l2 else headA
    return l1

def main():
    shared = ListNode(8, ListNode(4, ListNode(5)))
    headA = ListNode(4, ListNode(1, shared))
    headB = ListNode(5, ListNode(6, ListNode(1, shared)))
    assert (
        list_node_to_string(getIntersectionNode(None, headA, headB)) == "8->4->5"
    )

    headC = ListNode(2, ListNode(6, ListNode(4)))
    headD = ListNode(1, ListNode(5))
    assert list_node_to_string(getIntersectionNode(None, headC, headD)) == "None"

if __name__ == "__main__":
    main()
