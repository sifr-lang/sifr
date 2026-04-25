
# LeetCode 148: Sort List
# Python version
from helpers.list_node import ListNode, list_node_to_string
def sortList(head: ListNode | None) -> ListNode | None:
    if not head or not head.next:
        return head

    mid = get_mid(head)
    left, right = sortList(head), sortList(mid)

    return merge_two_sorted(left, right)

def merge_two_sorted(list1: ListNode | None, list2: ListNode | None) -> ListNode | None:
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

def get_mid(head: ListNode | None) -> ListNode | None:
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
