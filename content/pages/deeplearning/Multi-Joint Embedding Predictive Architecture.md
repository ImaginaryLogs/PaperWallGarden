---
aliases: M-JEPA
tags:
  - joint-embedding-predictive-architecture
  - deep-learning
date: 2026-06-02 01:01
---


A variation of the parent [[Joint Embedding Predictive Architecture]] where the student and teacher networks process completely different types of data (modalities) from the same problem.

Instead of masking a single image, you feed Modality A (e.g., structural anatomy from a CT scan) to the Student, and Modality B (e.g., metabolic activity from a PET scan, or written text from a pathologist's report) to the Teacher. The predictor must use the anatomical features to predict the metabolic or text embedding space.

## The Disconnected Data Problem

Medical diagnosis rarely relies on a single image type. However, standard deep learning models struggle to fuse wildly different data streams without collapsing one or requiring massive, perfectly curated datasets. Multi-JEPA aligns distinct data streams into a unified "concept space" without needing to generate images from text or vice versa.