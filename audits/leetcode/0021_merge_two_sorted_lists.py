
# LeetCode 21: Merge Two Sorted Lists
# Python version
from helpers.list_node import ListNode, list_node_to_string
def mergeTwoLists(list1: ListNode, list2: ListNode) -> ListNode:
    dummy = node = ListNode()

    while list1 and list2:
        if list1.val < list2.val:
            node.next = list1
            list1 = list1.next
        else:
            node.next = list2
            list2 = list2.next
        node = node.next

    node.next = list1 or list2

    return dummy.next

# Recursive

def mergeTwoLists(list1: ListNode | None, list2: ListNode | None) -> ListNode | None:
    if not list1:
        return list2
    if not list2:
        return list1
    lil, big = (list1, list2) if list1.val < list2.val else (list2, list1)
    lil.next = mergeTwoLists(lil.next, big)
    return lil

def main():
    assert list_node_to_string(mergeTwoLists(ListNode(1, ListNode(2, ListNode(4, None))), ListNode(1, ListNode(3, ListNode(4, None))))) == list_node_to_string(ListNode(1, ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(4, None)))))))
    assert mergeTwoLists(None, None) == None
    assert list_node_to_string(mergeTwoLists(None, ListNode(0, None))) == list_node_to_string(ListNode(0, None))

if __name__ == "__main__":
    main()
