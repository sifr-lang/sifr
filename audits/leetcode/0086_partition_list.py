
# LeetCode 86: Partition List
# Python version
from helpers.list_node import ListNode, list_node_to_string
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
