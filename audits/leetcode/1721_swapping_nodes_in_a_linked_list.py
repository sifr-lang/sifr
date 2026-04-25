
# LeetCode 1721: Swapping Nodes In A Linked List
# Python version
from helpers.list_node import ListNode, list_node_to_string
def swapNodes(head: ListNode | None, k: int) -> ListNode | None:
    right_pointer = head
    for _ in range(1, k):
        right_pointer = right_pointer.next
    left_kth_node = right_pointer

    left_pointer = head
    while right_pointer is not None:
        right_kth_node = left_pointer
        right_pointer = right_pointer.next
        left_pointer = left_pointer.next

    left_kth_node.val, right_kth_node.val = right_kth_node.val, left_kth_node.val
    return head

def main():
    assert list_node_to_string(swapNodes(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None))))), 2)) == list_node_to_string(ListNode(1, ListNode(4, ListNode(3, ListNode(2, ListNode(5, None))))))
    assert list_node_to_string(swapNodes(ListNode(7, ListNode(9, ListNode(6, ListNode(6, ListNode(7, ListNode(8, ListNode(3, ListNode(0, ListNode(9, ListNode(5, None)))))))))), 5)) == list_node_to_string(ListNode(7, ListNode(9, ListNode(6, ListNode(6, ListNode(8, ListNode(7, ListNode(3, ListNode(0, ListNode(9, ListNode(5, None)))))))))))

if __name__ == "__main__":
    main()
