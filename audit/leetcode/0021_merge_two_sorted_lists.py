# LeetCode 21: Merge Two Sorted Lists
# Python version

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

def mergeTwoLists(list1: Optional[ListNode], list2: Optional[ListNode]) -> Optional[ListNode]:
    if not list1:
        return list2
    if not list2:
        return list1
    lil, big = (list1, list2) if list1.val < list2.val else (list2, list1)
    lil.next = mergeTwoLists(lil.next, big)
    return lil



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
