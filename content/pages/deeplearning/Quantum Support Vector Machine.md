---
aliases:
  - qsvm
tags:
  - deep-learning
  - quantum-machine-learning
  - quantum-support-vector-machine
---

A non-neural network approach to QML that uses a quantum computer to calculate a custom "distance metric" between data points, which is then handled by a classical machine learning algorithm.

You compress your images into small feature vectors classically. You then use a Qiskit quantum feature map (like a `ZZFeatureMap`) to project those classical features into a massive, high-dimensional [[Hilbert space]] (quantum state space). In this massive space, you calculate how similar every image is to every other image. This creates a matrix of "quantum distances" (a Kernel Matrix). You hand this matrix over to a standard classical Support Vector Machine (SVM) running on your CPU to draw the final decision boundary.

# The Linear Inseparability Problem

In medical datasets, the subtle differences between an aggressive early-stage tumor and a benign cyst can be completely overlapping and impossible to separate linearly in standard dimensions. By projecting the features into a vast quantum state space, patterns that were completely tangled together suddenly untangle and become cleanly separable by a simple straight plane.