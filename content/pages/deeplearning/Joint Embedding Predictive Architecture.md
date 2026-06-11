---
aliases:
  - JEPA
tags:
  - joint-embedding-predictive-architecture
  - deep-learning
date: 2026-06-02 01:01
---


A self-supervised learning paradigm introduced by Yann LeCun of Meta AI. Instead of teaching a model to:
- Predict missing pixels (like a [[Masked Autoencoder]]), or
- Matching identical images (like a [[Contrastive Learning]]),
JEPA teaches a model to predict the abstract, high-level features of a missing piece of an image with a latent (hidden) embedding space.

## How it Works
It has a student and teacher network. Take an image, chop it in many different styles and patches, and mask a large portion of it. The student gets the unmasked context patches, while the teacher gets the whole image (including the masked targets). Between the two, a third lightweight network called a predictor takes the student's output and tries to guess what the brain state looks like for those missing target patches

This is the classical image-based JEPA. There's a variation of this.

## Pixel Waste and Unpredictability Problem

In medical imaging, generating pixels to create images is highly inefficient so it tends to be noisy.

If a CT scan has a noisy background or a slight artifact, a generative model (like a GAN or Diffusion model) wastes immense capacity trying to reconstruct that useless noise. 

JEPA ignores pixel-level noise, forcing the model to learn only the underlying semantic structures and anatomy.
