
# LeetCode 2: Add Two Numbers
# Python version
from helpers.list_node import ListNode, list_node_to_string
def addTwoNumbers(l1: ListNode, l2: ListNode) -> ListNode:
    dummy = ListNode()
    cur = dummy

    carry = 0
    while l1 or l2 or carry:
        v1 = l1.val if l1 else 0
        v2 = l2.val if l2 else 0

        # new digit
        val = v1 + v2 + carry
        carry = val // 10
        val = val % 10
        cur.next = ListNode(val)

        # update ptrs
        cur = cur.next
        l1 = l1.next if l1 else None
        l2 = l2.next if l2 else None

    return dummy.next

def main():
    assert list_node_to_string(
        addTwoNumbers(
            ListNode(2, ListNode(4, ListNode(3, None))),
            ListNode(5, ListNode(6, ListNode(4, None))),
        )
    ) == list_node_to_string(ListNode(7, ListNode(0, ListNode(8, None))))
    assert list_node_to_string(addTwoNumbers(ListNode(0, None), ListNode(0, None))) == list_node_to_string(ListNode(0, None))
    assert list_node_to_string(
        addTwoNumbers(
            ListNode(9, ListNode(9, ListNode(9, ListNode(9, ListNode(9, ListNode(9, ListNode(9, None))))))),
            ListNode(9, ListNode(9, ListNode(9, ListNode(9, None)))),
        )
    ) == list_node_to_string(
        ListNode(
            8,
            ListNode(
                9,
                ListNode(
                    9,
                    ListNode(
                        9,
                        ListNode(0, ListNode(0, ListNode(0, ListNode(1, None)))),
                    ),
                ),
            ),
        )
    )

if __name__ == "__main__":
    main()
