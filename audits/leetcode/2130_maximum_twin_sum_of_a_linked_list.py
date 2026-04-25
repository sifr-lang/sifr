
# LeetCode 2130: Maximum Twin Sum Of A Linked List
# Python version
from helpers.list_node import ListNode, list_node_to_string
def pairSum(head: ListNode | None) -> int:
    slow, fast = head, head
    prev = None
    while fast and fast.next:
        fast = fast.next.next
        tmp = slow.next
        slow.next = prev
        prev = slow
        slow = tmp

    res = 0
    while slow:
        res = max(res, prev.val + slow.val)
        prev = prev.next
        slow = slow.next
    return res

def main():
    assert pairSum(ListNode(5, ListNode(4, ListNode(2, ListNode(1, None))))) == 6
    assert pairSum(ListNode(4, ListNode(2, ListNode(2, ListNode(3, None))))) == 7
    assert pairSum(ListNode(1, ListNode(100000, None))) == 100001

if __name__ == "__main__":
    main()
