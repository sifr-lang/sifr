
# LeetCode 143: Reorder List
# Python version
from helpers.list_node import ListNode, list_node_to_string
def reorderList(head: ListNode) -> None:
    # find middle
    slow, fast = head, head.next
    while fast and fast.next:
        slow = slow.next
        fast = fast.next.next

    # reverse second half
    second = slow.next
    prev = slow.next = None
    while second:
        tmp = second.next
        second.next = prev
        prev = second
        second = tmp

    # merge two halfs
    first, second = head, prev
    while second:
        tmp1, tmp2 = first.next, second.next
        first.next = second
        second.next = tmp1
        first, second = tmp1, tmp2

def main():
    arg0 = ListNode(1, ListNode(2, ListNode(3, ListNode(4, None))))
    _result = reorderList(arg0)
    assert list_node_to_string(arg0) == list_node_to_string(ListNode(1, ListNode(4, ListNode(2, ListNode(3, None)))))
    arg0 = ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None)))))
    _result = reorderList(arg0)
    assert list_node_to_string(arg0) == list_node_to_string(ListNode(1, ListNode(5, ListNode(2, ListNode(4, ListNode(3, None))))))

if __name__ == "__main__":
    main()
