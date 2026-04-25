
# LeetCode 876: Middle Of The Linked List
# Python version
from helpers.list_node import ListNode, list_node_to_string
def middleNode(head: ListNode | None) -> ListNode | None:
    if not head or not head.next:
        return head

    slow = fast = head
    while fast and fast.next:
        slow, fast = slow.next, fast.next.next

    return slow

def main():
    assert list_node_to_string(middleNode(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None))))))) == list_node_to_string(ListNode(3, ListNode(4, ListNode(5, None))))
    assert list_node_to_string(middleNode(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, ListNode(6, None)))))))) == list_node_to_string(ListNode(4, ListNode(5, ListNode(6, None))))

if __name__ == "__main__":
    main()
