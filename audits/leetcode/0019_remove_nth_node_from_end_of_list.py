
# LeetCode 19: Remove Nth Node From End Of List
# Python version
from helpers.list_node import ListNode, list_node_to_string
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
