
# LeetCode 25: Reverse Nodes In K Group
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

def reverseKGroup(head: ListNode, k: int) -> ListNode:
    dummy = ListNode(0, head)
    groupPrev = dummy

    while True:
        kth = getKth(groupPrev, k)
        if not kth:
            break
        groupNext = kth.next

        # reverse group
        prev, curr = kth.next, groupPrev.next
        while curr != groupNext:
            tmp = curr.next
            curr.next = prev
            prev = curr
            curr = tmp

        tmp = groupPrev.next
        groupPrev.next = kth
        groupPrev = tmp
    return dummy.next


def getKth(curr, k):
    while curr and k > 0:
        curr = curr.next
        k -= 1
    return curr



def main():
    assert list_node_to_string(reverseKGroup(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None))))), 2)) == list_node_to_string(ListNode(2, ListNode(1, ListNode(4, ListNode(3, ListNode(5, None))))))
    assert list_node_to_string(reverseKGroup(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None))))), 3)) == list_node_to_string(ListNode(3, ListNode(2, ListNode(1, ListNode(4, ListNode(5, None))))))

if __name__ == "__main__":
    main()