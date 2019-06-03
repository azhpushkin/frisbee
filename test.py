weights = [
    1, 1, 4, 2, 2, 14, 9, 0, 18, 7, 5, 18, 16,
    8, 13, 11, 12, 7, 20, 20, 11, 16, 2, 17, 5,
    11, 3, 14, 0, 6, 1, 20, 1, 10, 5, 7, 9, 8,
    0, 4, 2, 18, 13, 4, 2, 15, 13, 8, 5, 14,
    13, 20, 6, 7, 16, 2, 5, 17, 14, 2, 18, 18,
    2, 14, 11, 6, 16, 9, 12, 3, 15, 11, 2, 17,
    15, 18, 17, 4, 19, 17, 16, 1, 10, 9, 9, 20,
    18, 13, 18, 15, 6, 1, 4, 7, 4, 5, 11, 0, 9, 20
]
costs = [
    6, 4, 1, 1, 5, 5, 4, 5, 3, 2, 1, 6, 4, 1,
    4, 6, 5, 1, 1, 2, 3, 2, 5, 2, 1, 3, 5, 3,
    5, 1, 5, 5, 2, 5, 1, 3, 1, 2, 2, 4, 4, 2,
    6, 4, 1, 1, 2, 3, 1, 4, 6, 6, 3, 1, 4, 1,
    2, 2, 5, 2, 1, 4, 4, 6, 1, 6, 4, 4, 4, 2,
    2, 1, 3, 5, 2, 4, 6, 4, 2, 1, 3, 4, 5, 6,
    1, 6, 1, 3, 2, 3, 1, 4, 5, 3, 2, 2, 2, 2, 4, 5
]

amount = 4
weights = weights[:amount]
costs = costs[:amount]

total = sum(weights)

goal = total // 2  # can only take half


l = list(zip(weights, costs))


def get_cost(l):
    return sum(cost for _, cost in l)


def get_weight(l):
    return sum(weight for weight, _ in l)

best = []
best_cost = None


def check_set(items):
    global best_cost
    global best

    w = get_weight(items)
    c = get_cost(items)

    if best_cost is None:
        if w <= goal:
            print(2)
            best = items
            best_cost = c
    else:
        if w <= goal and c > best_cost:
            best = items
            best_cost = c


def run(items):
    if len(items) > 0:
        check_set(items)

    for i in range(len(items)):
        run(items[:i] + items[1+i:])

print(goal, l)
run(l)
print(best_cost, best)









