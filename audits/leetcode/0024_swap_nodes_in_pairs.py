
# LeetCode 24: Swap Nodes In Pairs
# Python version
from helpers.list_node import ListNode, list_node_to_string

def swapPairs(head: ListNode | None) -> ListNode | None:
    dummy = ListNode(0, head)
    prev, curr = dummy, head

    while curr and curr.next:
        # save ptrs
        nxtPair = curr.next.next
        second = curr.next

        # reverse this pair
        second.next = curr
        curr.next = nxtPair
        prev.next = second

        # update ptrs
        prev = curr
        curr = nxtPair

    return dummy.next

def main():
    assert list_node_to_string(swapPairs(ListNode(1, ListNode(2, ListNode(3, ListNode(4, None)))))) == list_node_to_string(ListNode(2, ListNode(1, ListNode(4, ListNode(3, None)))))
    assert swapPairs(None) == None
    assert list_node_to_string(swapPairs(ListNode(1, None))) == list_node_to_string(ListNode(1, None))

if __name__ == "__main__":
    main()
