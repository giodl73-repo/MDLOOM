# ML Model Taxonomy

## By learning paradigm

mdloom:bullets
- Supervised Learning
  - Classification: Binary, Multi-class, Multi-label
  - Regression: Linear, Polynomial, Gaussian Process
  - Sequence-to-sequence: Machine translation, Summarization
- Unsupervised Learning
  - Clustering: K-means, DBSCAN, Hierarchical
  - Dimensionality reduction: PCA, t-SNE, UMAP
  - Generative: VAE, GAN, Diffusion
- Reinforcement Learning
  - Model-free: Q-learning, Policy gradient, Actor-critic
  - Model-based: Dyna, World models, MuZero

## Loss functions by task

mdloom:bullets
- Classification: Cross-entropy, Focal loss, Hinge
- Regression: MSE, MAE, Huber
- Generation: KL divergence, Wasserstein, Perceptual
- Sequence: CTC, Sequence cross-entropy, BLEU

## Bias-variance tradeoff

Key tradeoff: model capacity $\propto$ expressiveness, but also $\propto$ overfitting risk.

<!-- mdloom:compiled from="mdloom:math" -->
```
Error = Bias² + Variance + Irreducible noise
```
<!-- /mdloom:compiled -->
