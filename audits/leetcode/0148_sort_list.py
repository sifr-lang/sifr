# LeetCode 148: Sort List
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

def sortList(head: Optional[ListNode]) -> Optional[ListNode]:
    if not head or not head.next:
        return head

    mid = get_mid(head)
    left, right = sortList(head), sortList(mid)

    return merge_two_sorted(left, right)



def merge_two_sorted(list1: Optional[ListNode], list2: Optional[ListNode]) -> Optional[ListNode]:
    if not list1:
        return list2

    if not list2:
        return list1

    sentinel = ListNode()
    prev = sentinel
    while list1 and list2:
        if list1.val < list2.val:
            prev.next = list1
            list1 = list1.next
        else:
            prev.next = list2
            list2 = list2.next
        prev = prev.next

    if list1:
        prev.next = list1
    else:
        prev.next = list2

    return sentinel.next



def get_mid(head: Optional[ListNode]) -> Optional[ListNode]:
    mid_prev = None
    while head and head.next:
        mid_prev = mid_prev.next if mid_prev else head
        head = head.next.next

    mid = mid_prev.next
    mid_prev.next = None

    return mid



def main():
    assert list_node_to_string(sortList(ListNode(4, ListNode(2, ListNode(1, ListNode(3, None)))))) == list_node_to_string(ListNode(1, ListNode(2, ListNode(3, ListNode(4, None)))))
    assert list_node_to_string(sortList(ListNode(-1, ListNode(5, ListNode(3, ListNode(4, ListNode(0, None))))))) == list_node_to_string(ListNode(-1, ListNode(0, ListNode(3, ListNode(4, ListNode(5, None))))))
    assert sortList(None) == None

if __name__ == "__main__":
    main()