
# LeetCode 61: Rotate List
# Python version
from helpers.list_node import ListNode, list_node_to_string
def rotateRight(head: ListNode | None, k: int) -> ListNode | None:
    if not head or not head.next or k == 0:
        return head

    old_head = head

    curr, size = head, 0
    while curr:
        curr, size = curr.next, size + 1

    if k % size == 0:
        return head

    k %= size
    slow = fast = head
    while fast and fast.next:
        if k <= 0:
            slow = slow.next
        fast = fast.next
        k -= 1

    new_tail, new_head, old_tail = slow, slow.next, fast
    new_tail.next, old_tail.next = None, old_head

    return new_head

def main():
    assert list_node_to_string(rotateRight(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None))))), 2)) == list_node_to_string(ListNode(4, ListNode(5, ListNode(1, ListNode(2, ListNode(3, None))))))
    assert list_node_to_string(rotateRight(ListNode(0, ListNode(1, ListNode(2, None))), 4)) == list_node_to_string(ListNode(2, ListNode(0, ListNode(1, None))))

if __name__ == "__main__":
    main()
