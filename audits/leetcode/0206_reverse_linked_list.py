
# LeetCode 206: Reverse Linked List
# Python version
from helpers.list_node import ListNode, list_node_to_string

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
