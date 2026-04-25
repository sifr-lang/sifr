
# LeetCode 83: Remove Duplicates From Sorted List
# Python version
from helpers.list_node import ListNode, list_node_to_string
def deleteDuplicates(head: ListNode | None) -> ListNode | None:
    cur = head
    while cur:
        while cur.next and cur.next.val == cur.val:
            cur.next = cur.next.next
        cur = cur.next
    return head

def main():
    assert list_node_to_string(deleteDuplicates(ListNode(1, ListNode(1, ListNode(2, None))))) == list_node_to_string(ListNode(1, ListNode(2, None)))
    assert list_node_to_string(deleteDuplicates(ListNode(1, ListNode(1, ListNode(2, ListNode(3, ListNode(3, None))))))) == list_node_to_string(ListNode(1, ListNode(2, ListNode(3, None))))

if __name__ == "__main__":
    main()
