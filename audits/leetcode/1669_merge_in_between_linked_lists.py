
# LeetCode 1669: Merge In Between Linked Lists
# Python version
from helpers.list_node import ListNode, list_node_to_string
def mergeInBetween(list1: ListNode, a: int, b: int, list2: ListNode) -> ListNode:
    curr = list1
    i = 0
    while i < a - 1:
        curr = curr.next
        i += 1

    head = curr
    while i <= b:
        curr = curr.next
        i += 1
    head.next = list2

    while list2.next:
        list2 = list2.next
    list2.next = curr
    return list1

def main():
    assert list_node_to_string(mergeInBetween(ListNode(10, ListNode(1, ListNode(13, ListNode(6, ListNode(9, ListNode(5, None)))))), 3, 4, ListNode(1000000, ListNode(1000001, ListNode(1000002, None))))) == list_node_to_string(ListNode(10, ListNode(1, ListNode(13, ListNode(1000000, ListNode(1000001, ListNode(1000002, ListNode(5, None))))))))
    assert list_node_to_string(mergeInBetween(ListNode(0, ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, ListNode(6, None))))))), 2, 5, ListNode(1000000, ListNode(1000001, ListNode(1000002, ListNode(1000003, ListNode(1000004, None))))))) == list_node_to_string(ListNode(0, ListNode(1, ListNode(1000000, ListNode(1000001, ListNode(1000002, ListNode(1000003, ListNode(1000004, ListNode(6, None)))))))))

if __name__ == "__main__":
    main()
