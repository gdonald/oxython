class GrandParent:
    def __init__(self):
        self.level = "GrandParent"

class Parent(GrandParent):
    def __init__(self):
        super().__init__()
        self.parent_level = "Parent"

class Child(Parent):
    def __init__(self):
        super().__init__()
        self.child_level = "Child"

child = Child()
print(child.level)
print(child.parent_level)
print(child.child_level)
