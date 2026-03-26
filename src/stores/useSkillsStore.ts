import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SkillInfo } from "@/interfaces";

export const useSkillsStore = defineStore("skills", () => {
  const skills = ref<SkillInfo[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const selectedSkillId = ref<string | null>(null);

  const selectedSkill = computed(
    () => skills.value.find((skill) => skill.id === selectedSkillId.value) ?? null
  );

  async function loadSkills(force = false) {
    if (!force && skills.value.length > 0) return;

    loading.value = true;
    error.value = null;
    try {
      skills.value = await invoke<SkillInfo[]>("list_skills");
      if (selectedSkillId.value && !skills.value.some((skill) => skill.id === selectedSkillId.value)) {
        selectedSkillId.value = null;
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  function selectSkill(id: string | null) {
    selectedSkillId.value = id;
  }

  return {
    skills,
    loading,
    error,
    selectedSkillId,
    selectedSkill,
    loadSkills,
    selectSkill,
  };
});
