from sklearn.tree import DecisionTreeClassifier


def run() -> str:
    model = DecisionTreeClassifier(random_state=11)
    model.fit([[0, 0], [0, 1], [2, 2], [3, 2]], [0, 0, 1, 1])
    predictions = model.predict([[0, 0], [3, 3]]).tolist()
    classes = model.classes_.tolist()
    if predictions != [0, 1] or classes != [0, 1]:
        raise RuntimeError("scikit-learn full example returned unexpected predictions")
    return "sifr-python-interop:sklearn:predictions=0,1:classes=0,1"
